use core::cell::RefCell;
use defmt::Format;
use microbit::hal::spi::Spi;
use microbit::hal::Timer;
use microbit::pac::SPI0;
use rand::prelude::SmallRng;
use smart_leds::colors::{
    BLUE, DARK_GREEN, DARK_RED, DARK_TURQUOISE, GOLD, GREEN, INDIGO, MIDNIGHT_BLUE, PURPLE, RED,
    WHITE,
};
use smart_leds::RGB8;
use ws2812_spi::Ws2812;

pub(crate) mod carrousel;
pub(crate) mod double_carrousel;
pub(crate) mod forward_wave;
pub(crate) mod multi_color_fade_in;
pub(crate) mod multi_color_heartbeat;
pub(crate) mod multi_color_solid;
pub(crate) mod multi_color_solid_random;
pub(crate) mod multi_color_sparkle;
pub(crate) mod multi_color_strand;
pub(crate) mod uni_color_fade_in;
pub(crate) mod uni_color_front_to_back_wave;
pub(crate) mod uni_color_heartbeat;
pub(crate) mod uni_color_solid;
pub(crate) mod uni_color_sparkle;

pub const NUM_COLORS: usize = 11;
pub const NUM_LEDS: usize = 96 * 10;

pub(crate) trait Animation {
    fn brightness(&self, settings: &Settings) -> f32;

    fn render(
        &mut self, ws2812: &mut Ws2812<Spi<SPI0>>, timer: &mut Timer<microbit::pac::TIMER0>,
        settings: &Settings,
    );

    fn reset(&mut self);
}

pub(crate) const COLORS: [RGB8; NUM_COLORS] = [
    WHITE,
    RED,
    DARK_RED,
    GOLD,
    GREEN,
    DARK_GREEN,
    DARK_TURQUOISE,
    BLUE,
    MIDNIGHT_BLUE,
    PURPLE,
    INDIGO,
];

const NUM_STRANDS: usize = NUM_LEDS / 7;
const SHORTEST_DELAY: u32 = 5;

#[rustfmt::skip]
const VERTICAL_SLICES: [[Option<u16>; 152]; 16] = [
    // Slice 1
    [
        Some(175), Some(176), Some(177), Some(178), Some(179), Some(180), Some(181), Some(182),
        Some(183), Some(184), Some(185), Some(186), Some(187), Some(188), Some(189), Some(190),
        Some(191), Some(192), Some(193), Some(194), Some(195), Some(196), Some(197), Some(198),
        Some(199), Some(200), Some(201), Some(202), Some(203), Some(204),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None,
    ],
    // Slice 2
    [
        Some(153), Some(154), Some(155), Some(156), Some(157), Some(158), Some(159), Some(160),
        Some(161), Some(162), Some(163), Some(164), Some(165), Some(166), Some(167), Some(168),
        Some(169), Some(170), Some(171), Some(172), Some(173), Some(174),
        //
        Some(205), Some(206), Some(207), Some(208), Some(209), Some(210), Some(211), Some(212),
        Some(213), Some(214), Some(215), Some(216), Some(217), Some(218), Some(219), Some(220),
        Some(221), Some(222), Some(223), Some(224), Some(225), Some(226), Some(227), Some(228),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None,
    ],
    // Slice 3
    [
        Some(121), Some(122), Some(123), Some(124), Some(125), Some(126), Some(127), Some(128),
        Some(129), Some(130), Some(131), Some(132), Some(133), Some(134), Some(135), Some(136),
        Some(137), Some(138), Some(139), Some(140), Some(141), Some(142), Some(143), Some(144),
        Some(145), Some(146), Some(147), Some(148), Some(149), Some(150), Some(151), Some(152),
        //
        Some(229), Some(230), Some(231), Some(232), Some(233), Some(234), Some(235), Some(236),
        Some(237), Some(238), Some(239), Some(240), Some(241), Some(242), Some(243), Some(244),
        Some(245), Some(246), Some(247), Some(248), Some(249), Some(250), Some(251), Some(252),
        Some(253), Some(254), Some(255),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None,
    ],
    // Slice 4
    [
        Some(98), Some(99), Some(100), Some(101), Some(102), Some(103), Some(104), Some(105),
        Some(106), Some(107), Some(108), Some(109), Some(110), Some(111), Some(112), Some(113),
        Some(114), Some(115), Some(116), Some(117), Some(118), Some(119), Some(120),
        //
        Some(256), Some(257), Some(258), Some(259), Some(260), Some(261), Some(262), Some(263),
        Some(264), Some(265), Some(266), Some(267), Some(268), Some(269), Some(270), Some(271),
        Some(272), Some(273), Some(274), Some(275),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None,
    ],
    // Slice 5
    [
        Some(68), Some(69), Some(70), Some(71), Some(72), Some(73), Some(74), Some(75), Some(76),
        Some(77), Some(78), Some(79), Some(80), Some(81), Some(82), Some(83), Some(84), Some(85),
        Some(86), Some(87), Some(88), Some(89), Some(90), Some(91), Some(92), Some(93), Some(94),
        Some(95), Some(96), Some(97),
        //
        Some(276), Some(277), Some(278), Some(279), Some(280), Some(281), Some(282), Some(283),
        Some(284), Some(285), Some(286), Some(287), Some(288), Some(289), Some(290), Some(291),
        Some(292), Some(293), Some(294), Some(295), Some(296), Some(297), Some(298), Some(299),
        Some(300), Some(301), Some(302), Some(303), Some(304), Some(305), Some(306), Some(307),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Slice 6
    [
        Some(48), Some(49), Some(50), Some(51), Some(52), Some(53), Some(54), Some(55), Some(56),
        Some(57), Some(58), Some(59), Some(60), Some(61), Some(62), Some(63), Some(64), Some(65),
        Some(66), Some(67),
        //
        Some(308), Some(309), Some(310), Some(311), Some(312), Some(313), Some(314), Some(315),
        Some(316), Some(317), Some(318), Some(319), Some(320), Some(321), Some(322), Some(323),
        Some(324), Some(325),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None,
    ],
    // Slice 7
    [
        Some(19), Some(20), Some(21), Some(22), Some(23), Some(24), Some(25), Some(26), Some(27),
        Some(28), Some(29), Some(30), Some(31), Some(32), Some(33), Some(34), Some(35), Some(36),
        Some(37), Some(38), Some(39), Some(40), Some(41), Some(42), Some(43), Some(44), Some(45),
        Some(46), Some(47),
        //
        Some(326), Some(327), Some(328), Some(329), Some(330), Some(331), Some(332), Some(333),
        Some(334), Some(335), Some(336), Some(337), Some(338), Some(339), Some(340), Some(341),
        Some(342), Some(343), Some(344),
        //
        Some(350), Some(351), Some(352), Some(353), Some(354),
        //
        Some(483), Some(484), Some(485), Some(486), Some(487), Some(488), Some(489), Some(490),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None,
    ],
    // Slice 8
    [
        Some(2), Some(3), Some(4), Some(5), Some(6), Some(7), Some(8), Some(9), Some(10), Some(11),
        Some(12), Some(13), Some(14), Some(15), Some(16), Some(17), Some(18),
        //
        Some(345), Some(346), Some(347), Some(348), Some(349),
        //
        Some(355), Some(356), Some(357), Some(358), Some(359), Some(360), Some(361), Some(362),
        Some(363), Some(364), Some(365), Some(366), Some(367), Some(368), Some(369), Some(370),
        Some(371), Some(372), Some(373), Some(374), Some(375), Some(376), Some(377), Some(378),
        Some(379), Some(380), Some(381), Some(382), Some(383),
        //
        Some(476), Some(477), Some(478), Some(479), Some(480), Some(481), Some(482),
        //
        Some(491), Some(492), Some(493), Some(494), Some(495), Some(496), Some(497),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Slice 9
    [
        Some(0), Some(1),
        //
        Some(384), Some(385), Some(386), Some(387), Some(388), Some(389), Some(390),
        //
        Some(464), Some(465), Some(466), Some(467), Some(468), Some(469), Some(470), Some(471),
        Some(472), Some(473), Some(474), Some(475),
        //
        Some(498), Some(499), Some(500), Some(501), Some(502), Some(503), Some(504), Some(505),
        Some(506), Some(507), Some(508), Some(509),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Slice 10
    [
        Some(391), Some(392), Some(393), Some(394), Some(395), Some(396), Some(397), Some(398),
        Some(399), Some(400), Some(401), Some(402),
        //
        Some(452), Some(453), Some(454), Some(455), Some(456), Some(457), Some(458), Some(459),
        Some(460), Some(461), Some(462), Some(463),
        //
        Some(510), Some(511), Some(512), Some(513), Some(514), Some(515), Some(516), Some(517),
        Some(518), Some(519), Some(520), Some(521),
        //
        Some(564), Some(565), Some(566),Some(567), Some(568), Some(569), Some(570), Some(571),
        Some(572), Some(573), Some(574), Some(575),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Slice 11
    [
        Some(403), Some(404), Some(405), Some(406), Some(407), Some(408), Some(409), Some(410),
        Some(411), Some(412), Some(413), Some(414), Some(415), Some(416), Some(417), Some(418),
        Some(419), Some(420), Some(421), Some(422),
        //
        Some(440), Some(441), Some(442), Some(443), Some(444), Some(445), Some(446), Some(447),
        Some(448), Some(449), Some(450), Some(451),
        //
        Some(522), Some(523), Some(524), Some(525), Some(526), Some(527), Some(528), Some(529),
        Some(530), Some(531), Some(532),
        //
        Some(549), Some(550), Some(551), Some(552), Some(553), Some(554), Some(555), Some(556),
        Some(557), Some(558), Some(559), Some(560), Some(561), Some(562), Some(563),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None,
    ],
    // Slice 12
    [
        Some(423), Some(424), Some(425), Some(426), Some(427), Some(428), Some(429), Some(430),
        Some(431), Some(432), Some(433), Some(434), Some(435), Some(436), Some(437), Some(438),
        Some(439),
        //
        Some(533), Some(534), Some(535), Some(536), Some(537), Some(538), Some(539), Some(540),
        Some(541), Some(542), Some(543), Some(544), Some(545), Some(546), Some(547), Some(548),
        //
        Some(576), Some(577),
        //
        Some(666), Some(667), Some(668), Some(669), Some(670), Some(671), Some(672), Some(673),
        Some(674), Some(675), Some(676), Some(677), Some(678), Some(679), Some(680), Some(681),
        Some(682), Some(683), Some(684), Some(685), Some(686), Some(687), Some(688), Some(689),
        Some(690), Some(691), Some(692), Some(693), Some(694), Some(695), Some(696), Some(697),
        Some(698), Some(699), Some(700), Some(701), Some(702), Some(703), Some(704),
        //
        Some(790), Some(791), Some(792), Some(793), Some(794), Some(795), Some(796), Some(797),
        Some(798), Some(799), Some(800), Some(801), Some(802), Some(803), Some(804), Some(805),
        Some(806), Some(807), Some(808), Some(809), Some(810), Some(811), Some(812), Some(813),
        Some(814), Some(815), Some(816), Some(817), Some(818), Some(819), Some(820), Some(821),
        Some(822), Some(823), Some(824), Some(825), Some(826), Some(827), Some(828), Some(829),
        Some(830),
        //
        Some(923), Some(924), Some(925), Some(926), Some(927), Some(928), Some(929), Some(930),
        Some(931), Some(932), Some(933), Some(934), Some(935), Some(936), Some(937), Some(938),
        Some(939), Some(940), Some(941), Some(942), Some(943), Some(944), Some(945), Some(946),
        Some(947), Some(948), Some(949), Some(950), Some(951), Some(952), Some(953), Some(954),
        Some(955), Some(956), Some(957), Some(958), Some(959),
    ],
    // Slice 13
    [
        Some(578), Some(579), Some(580), Some(581), Some(582), Some(583), Some(584), Some(585),
        //
        Some(658), Some(659), Some(660), Some(661), Some(662), Some(663), Some(664), Some(665),
        //
        Some(705), Some(706), Some(707), Some(708), Some(709), Some(710), Some(711), Some(712),
        //
        Some(782), Some(783), Some(784), Some(785), Some(786), Some(787), Some(788), Some(789),
        //
        Some(831), Some(832), Some(833), Some(834), Some(835), Some(836), Some(837), Some(838),
        //
        Some(914), Some(915), Some(916), Some(917), Some(918), Some(919), Some(920), Some(921),
        Some(922),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None,
    ],
    // Slice 14
    [
        Some(586), Some(587), Some(588), Some(589), Some(590), Some(591), Some(592), Some(593),
        Some(594),
        //
        Some(649), Some(650), Some(651), Some(652), Some(653), Some(654), Some(655), Some(656),
        Some(657),
        //
        Some(713), Some(714), Some(715), Some(716), Some(717), Some(718), Some(719), Some(720),
        Some(721),
        //
        Some(773), Some(774), Some(775), Some(776), Some(777), Some(778), Some(779), Some(780),
        Some(781),
        //
        Some(839), Some(840), Some(841), Some(842), Some(843), Some(844), Some(845), Some(846),
        Some(847),
        //
        Some(906), Some(907), Some(908), Some(909), Some(910), Some(911), Some(912), Some(913),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None,
    ],
    // Slice 15
    [
        Some(595), Some(596), Some(597), Some(598), Some(599), Some(600), Some(601), Some(602),
        //
        Some(641), Some(642), Some(643), Some(644), Some(645), Some(646), Some(647), Some(648),
        //
        Some(722), Some(723), Some(724), Some(725), Some(726), Some(727), Some(728), Some(729),
        //
        Some(768), Some(769), Some(770), Some(771), Some(772),
        //
        Some(848), Some(849), Some(850), Some(851), Some(852), Some(853), Some(854), Some(855),
        //
        Some(898), Some(899), Some(900), Some(901), Some(902), Some(903), Some(904), Some(905),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None,
    ],
    // Slice 16
    [
        Some(603), Some(604), Some(605), Some(606), Some(607), Some(608), Some(609), Some(610),
        Some(611), Some(612), Some(613), Some(614), Some(615), Some(616), Some(617), Some(618),
        Some(619), Some(620), Some(621), Some(622), Some(623), Some(624), Some(625), Some(626),
        Some(627), Some(628), Some(629), Some(630), Some(631), Some(632), Some(633), Some(634),
        Some(635), Some(636), Some(637), Some(638), Some(639), Some(640),
        //
        Some(730), Some(731), Some(732), Some(733), Some(734), Some(735), Some(736), Some(737),
        Some(738), Some(739), Some(740), Some(741), Some(742), Some(743), Some(744), Some(745),
        Some(746), Some(747), Some(748), Some(749), Some(750), Some(751), Some(752), Some(753),
        Some(754), Some(755), Some(756), Some(757), Some(758), Some(759), Some(760), Some(761),
        Some(762), Some(763), Some(764), Some(765), Some(766), Some(767),
        //
        Some(856), Some(857), Some(858), Some(859), Some(860), Some(861), Some(862), Some(863),
        Some(864), Some(865), Some(866), Some(867), Some(868), Some(869), Some(870), Some(871),
        Some(872), Some(873), Some(874), Some(875), Some(876), Some(877), Some(878), Some(879),
        Some(880), Some(881), Some(882), Some(883), Some(884), Some(885), Some(886), Some(887),
        Some(888), Some(889), Some(890), Some(891), Some(892), Some(893), Some(894), Some(895),
        Some(896), Some(897),
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None,
    ],
];

#[derive(Clone, Copy, Debug, Format)]
pub struct Settings {
    pub brightness: f32,
    pub color_index: usize,
    pub delay: u32,
}

impl Settings {
    pub fn new(color_index: usize, brightness: f32, delay: u32) -> Self {
        Settings {
            brightness,
            color_index,
            delay,
        }
    }

    pub fn set_brightness(&mut self, brightness: f32) {
        self.brightness = brightness;
    }

    pub fn set_color_index(&mut self, color_index: usize) {
        self.color_index = color_index;
    }

    pub fn set_delay(&mut self, delay: u32) {
        self.delay = delay;
    }
}

pub(crate) struct Carrousel<'a> {
    color_index: usize,
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    position: usize,
    prng: SmallRng,
}

pub(crate) struct DoubleCarrousel<'a> {
    color_index_1: usize,
    color_index_2: usize,
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    position_1: usize,
    position_2: usize,
    prng: SmallRng,
}

pub(crate) struct ForwardWave<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    position: usize,
    wrapped: bool,
}

pub(crate) struct MultiColorFadeIn<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    ascending: bool,
    color_index: usize,
    prng: SmallRng,
    current_step: u8,
}

pub(crate) struct MultiColorHeartbeat<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    color_index: usize,
    prng: SmallRng,
    current_step: u8,
    sequence: u8,
}

pub(crate) struct MultiColorSolid<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
}

pub(crate) struct MultiColorSolidRandom<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
    rendered_data: [RGB8; NUM_LEDS],
}

pub(crate) struct MultiColorSparkle<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
}

pub(crate) struct MultiColorStrand<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
    strands: [Strand; NUM_STRANDS],
}

#[derive(Clone, Copy, Default)]
struct Strand {
    color_index: u8,
    start: u16,
    end: u16,
    position: u16,
}

pub(crate) struct UniColorFadeIn<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    ascending: bool,
    current_step: u8,
}

pub(crate) struct UniColorFrontToBackWave<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    position: usize,
}

pub(crate) struct UniColorHeartbeat<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    current_step: u8,
    sequence: u8,
}

pub(crate) struct UniColorSolid<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
}

pub(crate) struct UniColorSparkle<'a> {
    data: &'a RefCell<[RGB8; NUM_LEDS]>,
    prng: SmallRng,
}

fn create_color_with_brightness(color: &RGB8, brightness: f32) -> RGB8 {
    RGB8::new(
        (color.r as f32 * brightness) as u8,
        (color.g as f32 * brightness) as u8,
        (color.b as f32 * brightness) as u8,
    )
}

fn reset_data(data: &RefCell<[RGB8; NUM_LEDS]>) {
    let mut data = data.borrow_mut();
    for i in 0..NUM_LEDS {
        data[i] = RGB8::default();
    }
}
